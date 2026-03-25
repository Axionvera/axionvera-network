#!/bin/bash

# Terraform deployment script for axionvera-network
set -e

echo "🚀 Starting Terraform deployment for axionvera-network..."

# Check if required tools are installed
command -v terraform >/dev/null 2>&1 || { echo "❌ Terraform is not installed. Please install it first."; exit 1; }
command -v aws >/dev/null 2>&1 || { echo "❌ AWS CLI is not installed. Please install it first."; exit 1; }

# Navigate to terraform directory
cd "$(dirname "$0")/../terraform"

# Initialize Terraform
echo "📦 Initializing Terraform..."
terraform init

# Validate the configuration
echo "✅ Validating Terraform configuration..."
terraform validate

# Plan the deployment
echo "📋 Planning deployment..."
terraform plan -out=tfplan

# Ask for confirmation
echo "❓ Do you want to apply this plan? (y/N)"
read -r response
if [[ ! "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    echo "❌ Deployment cancelled."
    exit 0
fi

# Apply the configuration
echo "🏗️  Applying Terraform configuration..."
terraform apply tfplan

echo "✅ Terraform deployment completed successfully!"

# Display important outputs
echo ""
echo "📊 Deployment Summary:"
echo "======================"
terraform output

echo ""
echo "🔗 Access Information:"
echo "======================"
echo "Load Balancer DNS: $(terraform output -raw alb_dns_name 2>/dev/null || echo 'Not available')"
echo "VPC ID: $(terraform output -raw vpc_id 2>/dev/null || echo 'Not available')"
echo "CloudWatch Log Group: $(terraform output -raw cloudwatch_log_group_name 2>/dev/null || echo 'Not available')"

echo ""
echo "🔧 Next Steps:"
echo "============"
echo "1. Update your DNS records to point to the Load Balancer DNS"
echo "2. Monitor the deployment using the CloudWatch dashboard"
echo "3. Check the application logs in CloudWatch"
echo "4. Test the health endpoints"

echo ""
echo "🧹 Cleanup command:"
echo "=================="
echo "To destroy all resources, run: terraform destroy"
