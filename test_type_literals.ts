// Test for type literal detection and partial matching

// Type literals in variable declarations
let user: { name: string; email: string; age: number } = {
    name: "John",
    email: "john@example.com",
    age: 30
};

const person: { name: string; email: string; age: number } = {
    name: "Jane", 
    email: "jane@example.com",
    age: 25
};

// Type literals in function parameters
function processUser(data: { name: string; email: string; age: number }): void {
    console.log(data.name);
}

function handlePerson(person: { name: string; email: string; age: number }): void {
    console.log(person.email);
}

// Type literals in function returns
function getUser(): { name: string; email: string; age: number } {
    return { name: "Bob", email: "bob@example.com", age: 35 };
}

function fetchPerson(): { name: string; email: string; age: number } {
    return { name: "Alice", email: "alice@example.com", age: 28 };
}

// Partial matches - similar but not identical
function processProfile(profile: { 
    name: string; 
    email: string; 
    age: number;
    avatar?: string;  // Extra optional property
}): void {
    console.log(profile.name);
}

// Nested type literals
function processOrder(order: {
    id: string;
    user: { name: string; email: string };
    items: Array<{ sku: string; price: number }>;
}): void {
    console.log(order.id);
}

function handleOrder(data: {
    id: string;
    user: { name: string; email: string };  // Same nested structure
    items: { sku: string; price: number }[];  // Different array syntax but same type
}): void {
    console.log(data.id);
}

// Complex type literals with methods
interface UserActions {
    updateProfile(data: { 
        name?: string; 
        email?: string; 
        bio?: string 
    }): void;
    
    changePassword(params: {
        oldPassword: string;
        newPassword: string;
    }): boolean;
}

interface PersonActions {
    modifyProfile(info: { 
        name?: string; 
        email?: string; 
        bio?: string  // Same structure as updateProfile parameter
    }): void;
    
    resetPassword(data: {
        oldPassword: string;
        newPassword: string;  // Same structure as changePassword parameter
    }): boolean;
}

// Arrow functions with type literals
const createUser = (data: { name: string; email: string }): { id: string; name: string; email: string } => {
    return { id: "123", ...data };
};

const makePerson = (info: { name: string; email: string }): { id: string; name: string; email: string } => {
    return { id: "456", ...info };
};

// Type literals in class methods
class UserService {
    create(data: { name: string; email: string; password: string }): { success: boolean; user?: any } {
        return { success: true };
    }
    
    update(id: string, data: { name?: string; email?: string }): { success: boolean; message: string } {
        return { success: true, message: "Updated" };
    }
}

class PersonService {
    add(info: { name: string; email: string; password: string }): { success: boolean; user?: any } {
        return { success: true };
    }
    
    modify(id: string, info: { name?: string; email?: string }): { success: boolean; message: string } {
        return { success: true, message: "Modified" };
    }
}

// Different but partially overlapping
function processPayment(payment: {
    amount: number;
    currency: string;
    method: "card" | "bank";
    metadata?: { orderId: string; userId: string };
}): void {
    console.log(payment.amount);
}

function handleTransaction(transaction: {
    amount: number;
    currency: string;
    type: "debit" | "credit";  // Different property
    metadata?: { orderId: string; customerId: string };  // Slightly different nested structure
}): void {
    console.log(transaction.amount);
}