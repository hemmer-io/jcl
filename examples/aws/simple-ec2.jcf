# Example: Simple EC2 instance with configuration

environment "production" {
  provider "aws" {
    region = "us-west-2"
  }

  variables {
    instance_type = "t3.medium"
    app_version = "1.0.0"
  }

  tags {
    environment = "production"
    managed_by = "jcl"
  }
}

stack "web_server" {
  environment = env.production

  # Reference existing VPC (not managed by JCL)
  data "aws_vpc" "main" {
    tags = {
      name = "main-vpc"
    }
    lifecycle {
      managed = false
    }
  }

  # Reference existing subnet
  data "aws_subnet" "public" {
    vpc_id = data.aws_vpc.main.id
    tags = {
      tier = "public"
    }
    lifecycle {
      managed = false
    }
  }

  # Create security group (managed)
  resource "aws_security_group" "web" {
    name = "web-server-sg"
    description = "Security group for web server"
    vpc_id = data.aws_vpc.main.id

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

    tags = merge(env.tags, {
      name = "web-server-sg"
    })
  }

  # Create EC2 instance
  resource "aws_instance" "web" {
    ami = "ami-0c55b159cbfafe1f0"  # Amazon Linux 2
    instance_type = env.vars.instance_type
    subnet_id = data.aws_subnet.public.id
    vpc_security_group_ids = [resource.aws_security_group.web.id]

    tags = merge(env.tags, {
      name = "web-server"
      version = env.vars.app_version
    })

    # Configure the instance after creation
    configure {
      # Update system packages
      package "system-update" {
        state = "latest"
      }

      # Install nginx
      package "nginx" {
        state = "present"
      }

      # Configure nginx
      file "/etc/nginx/nginx.conf" {
        content = template("./nginx.conf.tpl")
        mode = "0644"
        owner = "root"
        group = "root"
      }

      # Create web content
      file "/usr/share/nginx/html/index.html" {
        content = "<h1>Hello from JCL v${env.vars.app_version}</h1>"
        mode = "0644"
      }

      # Start nginx service
      service "nginx" {
        state = "running"
        enabled = true
      }
    }
  }

  # Outputs
  output "instance_id" {
    value = resource.aws_instance.web.id
    description = "EC2 instance ID"
  }

  output "public_ip" {
    value = resource.aws_instance.web.public_ip
    description = "Public IP address"
  }

  output "web_url" {
    value = "http://${resource.aws_instance.web.public_ip}"
    description = "Web server URL"
  }
}
