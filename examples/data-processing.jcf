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
active_employees = [user for user in users if user.active]

# Get all engineering employees
engineers = [user for user in users if user.department == "Engineering"]

# Extract just names and salaries
names_and_salaries = [
    (
        name = user.name,
        salary = user.salary
    )
    for user in users
]

# Calculate salary statistics for active employees
active_salaries = [user.salary for user in active_employees]

salary_stats = (
    min = min(active_salaries),
    max = max(active_salaries),
    average = avg(active_salaries),
    total = sum(active_salaries),
    count = length(active_salaries)
)

# Group employees by department
departments = distinct([user.department for user in users])

employees_by_dept = [
    (
        department = dept,
        employees = [user.name for user in users if user.department == dept],
        count = length([user for user in users if user.department == dept]),
        avg_salary = avg([user.salary for user in users if user.department == dept])
    )
    for dept in departments
]

# Find high earners (salary > $80k)
high_earners = [
    (
        name = user.name,
        department = user.department,
        salary = user.salary
    )
    for user in users if user.salary > 80000
]

# Format employee directory with string functions
employee_directory = [
    format(
        "%s (%s) - %s",
        upper(user.name),
        user.department,
        format("$%d", user.salary)
    )
    for user in active_employees
]

# Generate email addresses
employee_emails = [
    (
        name = user.name,
        email = lower(replace(
            format("%s@example.com", user.name),
            " ",
            "."
        ))
    )
    for user in users
]

# Calculate department budget allocations
dept_budgets = [
    (
        department = dept_info.department,
        employee_count = dept_info.count,
        total_salaries = dept_info.avg_salary * dept_info.count,
        budget_per_person = dept_info.avg_salary,
        # Add 20% overhead for benefits
        total_budget = dept_info.avg_salary * dept_info.count * 1.2
    )
    for dept_info in employees_by_dept
]

# Summary report
summary_report = (
    total_employees = length(users),
    active_employees = length(active_employees),
    departments = length(departments),
    total_payroll = sum(active_salaries),
    average_salary = salary_stats.average,
    highest_paid = [
        user.name
        for user in users
        if user.salary == salary_stats.max
    ],
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
csv_rows = [
    format(
        "%d,%s,%s,%d,%b",
        user.id,
        user.name,
        user.department,
        user.salary,
        user.active
    )
    for user in users
]
csv_output = join(flatten([[csv_header], csv_rows]), "\n")
