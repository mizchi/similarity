// Test file for class similarity detection

// Similar classes with same structure
class User {
    name: string;
    email: string;
    age: number;
    
    constructor(name: string, email: string, age: number) {
        this.name = name;
        this.email = email;
        this.age = age;
    }
    
    getInfo(): string {
        return `${this.name} (${this.email})`;
    }
    
    updateEmail(newEmail: string): void {
        this.email = newEmail;
    }
}

class Person {
    name: string;
    email: string;
    age: number;
    
    constructor(name: string, email: string, age: number) {
        this.name = name;
        this.email = email;
        this.age = age;
    }
    
    getInfo(): string {
        return `${this.name} - ${this.email}`;
    }
    
    updateEmail(email: string): void {
        this.email = email;
    }
}

// Similar classes with inheritance
abstract class Animal {
    name: string;
    age: number;
    
    abstract makeSound(): string;
    
    getAge(): number {
        return this.age;
    }
}

class Dog extends Animal {
    breed: string;
    
    constructor(name: string, age: number, breed: string) {
        super();
        this.name = name;
        this.age = age;
        this.breed = breed;
    }
    
    makeSound(): string {
        return "Woof!";
    }
    
    wagTail(): boolean {
        return true;
    }
}

class Cat extends Animal {
    color: string;
    
    constructor(name: string, age: number, color: string) {
        super();
        this.name = name;
        this.age = age;
        this.color = color;
    }
    
    makeSound(): string {
        return "Meow!";
    }
    
    purr(): boolean {
        return true;
    }
}

// Different classes
class Product {
    sku: string;
    price: number;
    
    constructor(sku: string, price: number) {
        this.sku = sku;
        this.price = price;
    }
    
    applyDiscount(percentage: number): number {
        return this.price * (1 - percentage / 100);
    }
}

class Order {
    id: string;
    items: any[];
    total: number;
    
    constructor(id: string) {
        this.id = id;
        this.items = [];
        this.total = 0;
    }
    
    addItem(item: any): void {
        this.items.push(item);
    }
    
    calculateTotal(): number {
        return this.items.reduce((sum, item) => sum + item.price, 0);
    }
}