# VPC Module
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = "${local.project_name}-vpc"
  cidr = var.vpc_cidr

  azs             = data.aws_availability_zones.available.names
  private_subnets  = var.private_subnet_cidrs
  public_subnets   = var.public_subnet_cidrs

  enable_nat_gateway = true
  single_nat_gateway = false
  one_nat_gateway_per_az = true

  enable_dns_hostnames = true
  enable_dns_support   = true

  # Database subnet group (for future use)
  create_database_subnet_group = false
  create_database_subnet_route_table = false

  # Flow logs for security monitoring
  enable_flow_log = true
  flow_log_destination_type = "cloud-watch-logs"
  flow_log_log_group_name = "${local.project_name}-vpc-flow-logs"
  flow_log_cloudwatch_log_group_retention_in_days = 14

  tags = local.common_tags
}

# Security Groups
resource "aws_security_group" "network_node_sg" {
  name        = "${local.project_name}-network-node-sg"
  description = "Security group for axionvera network nodes"
  vpc_id      = module.vpc.vpc_id

  # RPC/API ports (example: 8000-8010)
  ingress {
    description = "RPC API access"
    from_port   = 8000
    to_port     = 8010
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # P2P network ports (example: 9000-9010)
  ingress {
    description = "P2P network communication"
    from_port   = 9000
    to_port     = 9010
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Health check ports
  ingress {
    description = "Health check endpoints"
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # SSH access (restricted)
  ingress {
    description = "SSH access"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = var.ssh_allowed_ips
  }

  # Egress rules - restrict external database access
  egress {
    description = "HTTP/HTTPS outbound"
    from_port   = 80
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    description = "DNS outbound"
    from_port   = 53
    to_port     = 53
    protocol    = "udp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    description = "NTP outbound"
    from_port   = 123
    to_port     = 123
    protocol    = "udp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Block external database access (ports 3306, 5432, 1433, 6379)
  egress {
    description = "Block MySQL access"
    from_port   = 3306
    to_port     = 3306
    protocol    = "tcp"
    cidr_blocks = []
  }

  egress {
    description = "Block PostgreSQL access"
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = []
  }

  egress {
    description = "Block Redis access"
    from_port   = 6379
    to_port     = 6379
    protocol    = "tcp"
    cidr_blocks = []
  }

  tags = local.common_tags
}

# Load Balancer Security Group
resource "aws_security_group" "alb_sg" {
  name        = "${local.project_name}-alb-sg"
  description = "Security group for Application Load Balancer"
  vpc_id      = module.vpc.vpc_id

  ingress {
    description = "HTTP"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "HTTPS"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    description = "All outbound"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = local.common_tags
}

# Data source for availability zones
data "aws_availability_zones" "available" {
  state = "available"
}
