# Example: Full web application stack with load balancer

environment "production" {
  provider "aws" {
    region = "us-west-2"
    account_id = "123456789012"
  }

  variables {
    app_name = "myapp"
    app_version = "2.1.0"
    instance_type = "t3.large"
    min_instances = 3
    max_instances = 10
    db_size = "large"
  }

  tags {
    environment = "production"
    application = "myapp"
    team = "platform"
    cost_center = "engineering"
  }
}

# Network stack (references existing infrastructure)
stack "network" {
  environment = env.production

  data "aws_vpc" "main" {
    id = "vpc-12345678"
    lifecycle { managed = false }
  }

  data "aws_subnet" "public" {
    for_each = ["us-west-2a", "us-west-2b", "us-west-2c"]

    vpc_id = data.aws_vpc.main.id
    availability_zone = each.value
    tags = { tier = "public" }

    lifecycle { managed = false }
  }

  data "aws_subnet" "private" {
    for_each = ["us-west-2a", "us-west-2b", "us-west-2c"]

    vpc_id = data.aws_vpc.main.id
    availability_zone = each.value
    tags = { tier = "private" }

    lifecycle { managed = false }
  }
}

# Application stack
stack "application" {
  environment = env.production
  depends_on = [stack.network]

  # Security group for ALB
  resource "aws_security_group" "alb" {
    name = "${env.vars.app_name}-alb-sg"
    vpc_id = stack.network.data.aws_vpc.main.id

    ingress {
      from_port = 80
      to_port = 80
      protocol = "tcp"
      cidr_blocks = ["0.0.0.0/0"]
    }

    ingress {
      from_port = 443
      to_port = 443
      protocol = "tcp"
      cidr_blocks = ["0.0.0.0/0"]
    }

    egress {
      from_port = 0
      to_port = 0
      protocol = "-1"
      cidr_blocks = ["0.0.0.0/0"]
    }

    tags = merge(env.tags, { name = "${env.vars.app_name}-alb-sg" })
  }

  # Security group for app servers
  resource "aws_security_group" "app" {
    name = "${env.vars.app_name}-app-sg"
    vpc_id = stack.network.data.aws_vpc.main.id

    ingress {
      from_port = 3000
      to_port = 3000
      protocol = "tcp"
      security_groups = [resource.aws_security_group.alb.id]
    }

    ingress {
      from_port = 22
      to_port = 22
      protocol = "tcp"
      cidr_blocks = ["10.0.0.0/8"]  # Internal only
    }

    egress {
      from_port = 0
      to_port = 0
      protocol = "-1"
      cidr_blocks = ["0.0.0.0/0"]
    }

    tags = merge(env.tags, { name = "${env.vars.app_name}-app-sg" })
  }

  # Application Load Balancer
  resource "aws_lb" "app" {
    name = "${env.vars.app_name}-alb"
    load_balancer_type = "application"
    subnets = stack.network.data.aws_subnet.public[*].id
    security_groups = [resource.aws_security_group.alb.id]

    tags = merge(env.tags, { name = "${env.vars.app_name}-alb" })
  }

  # Target group
  resource "aws_lb_target_group" "app" {
    name = "${env.vars.app_name}-tg"
    port = 3000
    protocol = "HTTP"
    vpc_id = stack.network.data.aws_vpc.main.id

    health_check {
      path = "/health"
      interval = 30
      timeout = 5
      healthy_threshold = 2
      unhealthy_threshold = 3
    }

    tags = merge(env.tags, { name = "${env.vars.app_name}-tg" })
  }

  # Listener
  resource "aws_lb_listener" "http" {
    load_balancer_arn = resource.aws_lb.app.arn
    port = 80
    protocol = "HTTP"

    default_action {
      type = "forward"
      target_group_arn = resource.aws_lb_target_group.app.arn
    }
  }

  # Launch template for app servers
  resource "aws_launch_template" "app" {
    name = "${env.vars.app_name}-lt"
    image_id = "ami-0c55b159cbfafe1f0"
    instance_type = env.vars.instance_type

    vpc_security_group_ids = [resource.aws_security_group.app.id]

    user_data = base64encode(template("./user-data.sh.tpl", {
      app_version = env.vars.app_version
    }))

    tags = merge(env.tags, { name = "${env.vars.app_name}-lt" })
  }

  # Auto Scaling Group
  resource "aws_autoscaling_group" "app" {
    name = "${env.vars.app_name}-asg"
    min_size = env.vars.min_instances
    max_size = env.vars.max_instances
    desired_capacity = env.vars.min_instances

    vpc_zone_identifier = stack.network.data.aws_subnet.private[*].id
    target_group_arns = [resource.aws_lb_target_group.app.arn]

    launch_template {
      id = resource.aws_launch_template.app.id
      version = "$Latest"
    }

    health_check_type = "ELB"
    health_check_grace_period = 300

    tag {
      key = "Name"
      value = "${env.vars.app_name}-instance"
      propagate_at_launch = true
    }
  }

  # Scaling policy based on CPU
  resource "aws_autoscaling_policy" "cpu" {
    name = "${env.vars.app_name}-cpu-scaling"
    autoscaling_group_name = resource.aws_autoscaling_group.app.name
    policy_type = "TargetTrackingScaling"

    target_tracking_configuration {
      predefined_metric_specification {
        predefined_metric_type = "ASGAverageCPUUtilization"
      }
      target_value = 70.0
    }
  }

  # Outputs
  output "alb_dns" {
    value = resource.aws_lb.app.dns_name
    description = "Load balancer DNS name"
  }

  output "app_url" {
    value = "http://${resource.aws_lb.app.dns_name}"
    description = "Application URL"
  }
}
