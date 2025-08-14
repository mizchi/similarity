// Test file for structure comparison framework

// Interface with common structure
interface User {
  id: string;
  name: string;
  email: string;
  age?: number;
}

// Type alias with similar structure
type Person = {
  id: string;
  name: string; 
  email: string;
  age?: number;
};

// Another interface with same properties (should be detected as similar)
interface Customer {
  id: string;
  name: string;
  email: string;
  age?: number;
}

// Type literal in variable declaration
const employee: {
  id: string;
  name: string;
  email: string;
  age?: number;
} = {
  id: "emp001",
  name: "John Doe",
  email: "john@example.com",
  age: 30
};

// Similar class structure
class Account {
  id: string;
  name: string;
  email: string;
  age?: number;

  constructor(id: string, name: string, email: string, age?: number) {
    this.id = id;
    this.name = name;
    this.email = email;
    this.age = age;
  }
}

// Slightly different structure (missing email)
interface Admin {
  id: string;
  name: string;
  role: string;
  age?: number;
}