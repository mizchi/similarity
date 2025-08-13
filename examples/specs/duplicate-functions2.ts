// Example: Function duplication detection

// Duplicate 1: Nearly identical functions (variable names only)
function calculateUserAge(birthYear: number): number {
  const currentYear = new Date().getFullYear();
  const age = currentYear - birthYear;
  return age;
}

function calculateCustomerAge(birthYear: number): number {
  const currentYear = new Date().getFullYear();
  const age = currentYear - birthYear;
  return age;
}

// Duplicate 2: Same algorithm, different implementation style
function findMaxValue(numbers: number[]): number {
  let max = numbers[0];
  for (let i = 1; i < numbers.length; i++) {
    if (numbers[i] > max) {
      max = numbers[i];
    }
  }
  return max;
}

function getMaximumValue(values: number[]): number {
  let maximum = values[0];
  for (const value of values) {
    if (value > maximum) {
      maximum = value;
    }
  }
  return maximum;
}

// Duplicate 3: Data processing with different field names
function processUserData(users: any[]) {
  return users
    .filter(user => user.isActive)
    .map(user => ({
      id: user.userId,
      name: user.fullName,
      email: user.emailAddress
    }));
}

function processCustomerData(customers: any[]) {
  return customers
    .filter(customer => customer.isActive)
    .map(customer => ({
      id: customer.customerId,
      name: customer.fullName,
      email: customer.emailAddress
    }));
}