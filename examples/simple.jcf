# Simple JCL Example - Web Application

# Structure
environments = (prod, dev)
stacks = (network, application)

# Production environment
env.prod = (
  region = us-west-2

  vars (
    app_name = myapp
    app_version = 1.2.3
    instance_type = t3.large
    count = 3
  )

  tags (
    team = platform
    env = production
    managed_by = jcl
  )
)

# Network stack - reference existing
stack.network = (
  env = prod
  resources = (vpc, subnet, sg)
)

read.vpc = (
  id = vpc-12345
)

read.subnet = (
  vpc = read.vpc.id
  tags (tier = public)
)

resource.sg = (
  vpc = read.vpc.id
  name = ${env.prod.vars.app_name}-sg

  allow (80, 443) from 0.0.0.0/0
)

# Application stack
stack.application = (
  env = prod
  depends_on = (network)
  resources = (web_instance, lb)
)

resource.web_instance = (
  ami = ami-12345
  type = env.prod.vars.instance_type
  count = env.prod.vars.count

  subnet = read.subnet.id
  security_groups = (resource.sg.id)

  tags (
    name = ${env.prod.vars.app_name}-web
    version = env.prod.vars.app_version
  )

  configure (
    install nginx nodejs

    file /etc/nginx/nginx.conf (
      from = template/nginx.conf
      mode = 0644
    )

    file /var/www/html/index.html (
      content = "<h1>App v${env.prod.vars.app_version}</h1>"
      mode = 0644
    )

    service nginx start enabled
  )
)

resource.lb = (
  type = application
  subnets = (read.subnet.id)
  security_groups = (resource.sg.id)

  listener (
    port = 80
    forward_to = resource.web_instance.*.id
  )

  health_check (
    path = /health
    interval = 30
  )
)

# Outputs
out.instance_ips = resource.web_instance.*.public_ip
out.lb_url = "http://${resource.lb.dns_name}"
