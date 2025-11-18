# Data Processing Pipeline Example
# Demonstrates list processing, filtering, and transformation

# Sample data: user records
users = [
    (id = 1, name = "Alice Johnson", age = 28, department = "Engineering", salary = 85000, active = true),
    (id = 2, name = "Bob Smith", age = 35, department = "Sales", salary = 65000, active = true),
    (id = 3, name = "Carol White", age = 42, department = "Engineering", salary = 95000, active = true),
    (id = 4, name = "David Brown", age = 31, department = "Marketing", salary = 70000, active = false),
    (id = 5, name = "Eve Davis", age = 29, department = "Engineering", salary = 88000, active = true),
    (id = 6, name = "Frank Miller", age = 38, department = "Sales", salary = 72000, active = true),
    (id = 7, name = "Grace Lee", age = 45, department = "Management", salary = 110000, active = true),
    (id = 8, name = "Henry Wilson", age = 27, department = "Engineering", salary = 80000, active = false)
]

# Filter active employees
active_employees = for user in users if user.active do user

# Get all engineering employees
engineers = for user in users if user.department == "Engineering" do user

# Extract just names and salaries
names_and_salaries = for user in users do (
    name = user.name,
    salary = user.salary
)

# Calculate salary statistics for active employees
active_salaries = for user in active_employees do user.salary

salary_stats = (
    min = min(active_salaries),
    max = max(active_salaries),
    average = avg(active_salaries),
    total = sum(active_salaries),
    count = length(active_salaries)
)

# Group employees by department
departments = distinct(for user in users do user.department)

employees_by_dept = for dept in departments do (
    department = dept,
    employees = for user in users if user.department == dept do user.name,
    count = length(for user in users if user.department == dept do user),
    avg_salary = avg(for user in users if user.department == dept do user.salary)
)

# Find high earners (salary > $80k)
high_earners = for user in users if user.salary > 80000 do (
    name = user.name,
    department = user.department,
    salary = user.salary
)

# Format employee directory with string functions
employee_directory = for user in active_employees do format(
    "%s (%s) - %s",
    upper(user.name),
    user.department,
    format("$%d", user.salary)
)

# Generate email addresses
employee_emails = for user in users do (
    name = user.name,
    email = lower(replace(
        format("%s@example.com", user.name),
        " ",
        "."
    ))
)

# Calculate department budget allocations
dept_budgets = for dept_info in employees_by_dept do (
    department = dept_info.department,
    employee_count = dept_info.count,
    total_salaries = dept_info.avg_salary * dept_info.count,
    budget_per_person = dept_info.avg_salary,
    # Add 20% overhead for benefits
    total_budget = dept_info.avg_salary * dept_info.count * 1.2
)

# Summary report
summary_report = (
    total_employees = length(users),
    active_employees = length(active_employees),
    departments = length(departments),
    total_payroll = sum(active_salaries),
    average_salary = salary_stats.average,
    highest_paid = (
        for user in users
        if user.salary == salary_stats.max
        do user.name
    ),
    engineering_headcount = length(engineers),
    engineering_percentage = round(
        (length(engineers) / length(users)) * 100
    )
)

# Export data in different formats
export_json = jsonencode(summary_report)
export_yaml = yamlencode(summary_report)

# Generate CSV header and rows
csv_header = "ID,Name,Department,Salary,Active"
csv_rows = for user in users do format(
    "%d,%s,%s,%d,%b",
    user.id,
    user.name,
    user.department,
    user.salary,
    user.active
)
csv_output = join([csv_header] + csv_rows, "\n")
