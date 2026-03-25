# EC2 Instances for Network Nodes
resource "aws_iam_instance_profile" "network_node_profile" {
  name = "${local.project_name}-network-node-profile"
  role = aws_iam_role.network_node_role.name
}

resource "aws_iam_role" "network_node_role" {
  name = "${local.project_name}-network-node-role"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })

  tags = local.common_tags
}

resource "aws_iam_role_policy_attachment" "network_node_ssm" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
  role       = aws_iam_role.network_node_role.name
}

resource "aws_iam_role_policy" "network_node_custom" {
  name = "${local.project_name}-network-node-custom"
  role = aws_iam_role.network_node_role.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "cloudwatch:PutMetricData",
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "*"
      }
    ]
  })
}

# Launch Template
resource "aws_launch_template" "network_node_template" {
  name_prefix   = "${local.project_name}-network-node-"
  image_id      = data.aws_ami.ubuntu.id
  instance_type = var.instance_type
  user_data     = base64encode(local.user_data)

  iam_instance_profile {
    arn = aws_iam_instance_profile.network_node_profile.arn
  }

  network_interfaces {
    associate_public_ip_address = false
    security_groups             = [aws_security_group.network_node_sg.id]
    subnet_id                   = module.vpc.private_subnets[0]
  }

  tag_specifications {
    resource_type = "instance"
    tags = merge(local.common_tags, {
      Name = "${local.project_name}-network-node"
    })
  }

  monitoring {
    enabled = true
  }
}

# Auto Scaling Group
resource "aws_autoscaling_group" "network_nodes" {
  name                = "${local.project_name}-network-nodes"
  vpc_zone_identifier = module.vpc.private_subnets
  desired_capacity    = 2
  max_size            = 6
  min_size            = 2

  launch_template {
    id      = aws_launch_template.network_node_template.id
    version = "$Latest"
  }

  target_group_arns = [aws_lb_target_group.network_nodes.arn]

  health_check_type         = "EC2"
  health_check_grace_period = 300

  tag {
    key                 = "Name"
    value               = "${local.project_name}-network-node"
    propagate_at_launch = true
  }

  tag {
    key                 = "Project"
    value               = local.project_name
    propagate_at_launch = true
  }

  tag {
    key                 = "Environment"
    value               = var.environment
    propagate_at_launch = true
  }
}

# Application Load Balancer
resource "aws_lb" "network_alb" {
  name               = "${local.project_name}-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb_sg.id]
  subnets            = module.vpc.public_subnets

  enable_deletion_protection = false

  tags = local.common_tags
}

# Target Group
resource "aws_lb_target_group" "network_nodes" {
  name     = "${local.project_name}-network-nodes"
  port     = 8080
  protocol = "HTTP"
  vpc_id   = module.vpc.vpc_id

  health_check {
    enabled             = true
    healthy_threshold   = 2
    interval            = 30
    matcher             = "200"
    path                = "/health"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    unhealthy_threshold = 2
  }

  tags = local.common_tags
}

# Load Balancer Listener
resource "aws_lb_listener" "network_http" {
  load_balancer_arn = aws_lb.network_alb.arn
  port              = "80"
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.network_nodes.arn
  }
}

# Auto Scaling Policies
resource "aws_autoscaling_policy" "scale_up" {
  name                   = "${local.project_name}-scale-up"
  scaling_adjustment     = 1
  adjustment_type        = "ChangeInCapacity"
  cooldown               = 300
  autoscaling_group_name = aws_autoscaling_group.network_nodes.name
}

resource "aws_autoscaling_policy" "scale_down" {
  name                   = "${local.project_name}-scale-down"
  scaling_adjustment     = -1
  adjustment_type        = "ChangeInCapacity"
  cooldown               = 300
  autoscaling_group_name = aws_autoscaling_group.network_nodes.name
}

# CloudWatch Alarms
resource "aws_cloudwatch_metric_alarm" "cpu_high" {
  alarm_name          = "${local.project_name}-cpu-high"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/EC2"
  period              = "120"
  statistic           = "Average"
  threshold           = "70"
  alarm_description   = "This metric monitors ec2 cpu utilization"
  alarm_actions       = [aws_autoscaling_policy.scale_up.arn]

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.network_nodes.name
  }

  tags = local.common_tags
}

resource "aws_cloudwatch_metric_alarm" "cpu_low" {
  alarm_name          = "${local.project_name}-cpu-low"
  comparison_operator = "LessThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/EC2"
  period              = "120"
  statistic           = "Average"
  threshold           = "20"
  alarm_description   = "This metric monitors ec2 cpu utilization"
  alarm_actions       = [aws_autoscaling_policy.scale_down.arn]

  dimensions = {
    AutoScalingGroupName = aws_autoscaling_group.network_nodes.name
  }

  tags = local.common_tags
}

# Data source for latest Ubuntu AMI
data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# User data for instances
locals {
  user_data = <<-EOF
#!/bin/bash
set -e

# Update system
apt-get update -y

# Install Docker
apt-get install -y docker.io docker-compose-plugin
systemctl enable docker
systemctl start docker

# Add ubuntu user to docker group
usermod -aG docker ubuntu

# Create app directory
mkdir -p /opt/axionvera-network
cd /opt/axionvera-network

# Pull and run the application
docker run -d \
  --name axionvera-network \
  --restart unless-stopped \
  -p 8080:8080 \
  -p 8000-8010:8000-8010 \
  -p 9000-9010:9000-9010 \
  axionvera-network:latest

# Setup health check
cat > /opt/axionvera-network/health-check.sh << 'HEALTH_EOF'
#!/bin/bash
if docker ps | grep -q axionvera-network; then
    echo "OK"
    exit 0
else
    echo "ERROR: Container not running"
    exit 1
fi
HEALTH_EOF

chmod +x /opt/axionvera-network/health-check.sh

# Setup log rotation for Docker logs
cat > /etc/logrotate.d/docker-containers << 'LOG_EOF'
/var/lib/docker/containers/*/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0644 root root
}
LOG_EOF

echo "Setup completed successfully"
EOF
}
