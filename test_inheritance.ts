// Test for inheritance and interface implementation similarity

// Base class and interface
interface IRepository<T> {
    findById(id: string): T | null;
    save(item: T): void;
    delete(id: string): boolean;
    findAll(): T[];
}

abstract class BaseRepository<T> {
    protected items: Map<string, T> = new Map();
    
    findById(id: string): T | null {
        return this.items.get(id) || null;
    }
    
    save(item: T & { id: string }): void {
        this.items.set(item.id, item);
    }
    
    delete(id: string): boolean {
        return this.items.delete(id);
    }
    
    findAll(): T[] {
        return Array.from(this.items.values());
    }
    
    abstract validate(item: T): boolean;
}

// Two different implementations that inherit from same base
class UserRepository extends BaseRepository<User> {
    validate(user: User): boolean {
        return user.email.includes('@');
    }
    
    findByEmail(email: string): User | null {
        for (const user of this.items.values()) {
            if (user.email === email) return user;
        }
        return null;
    }
}

class ProductRepository extends BaseRepository<Product> {
    validate(product: Product): boolean {
        return product.price > 0;
    }
    
    findBySku(sku: string): Product | null {
        for (const product of this.items.values()) {
            if (product.sku === sku) return product;
        }
        return null;
    }
}

// Two unrelated classes implementing same interface
class MemoryUserRepository implements IRepository<User> {
    private users: Map<string, User> = new Map();
    
    findById(id: string): User | null {
        return this.users.get(id) || null;
    }
    
    save(user: User): void {
        this.users.set(user.id, user);
    }
    
    delete(id: string): boolean {
        return this.users.delete(id);
    }
    
    findAll(): User[] {
        return Array.from(this.users.values());
    }
}

class FileUserRepository implements IRepository<User> {
    private cache: Map<string, User> = new Map();
    
    findById(id: string): User | null {
        // In real impl, would read from file
        return this.cache.get(id) || null;
    }
    
    save(user: User): void {
        // In real impl, would write to file
        this.cache.set(user.id, user);
    }
    
    delete(id: string): boolean {
        // In real impl, would delete from file
        return this.cache.delete(id);
    }
    
    findAll(): User[] {
        // In real impl, would read all from file
        return Array.from(this.cache.values());
    }
}

// Completely unrelated class (should not match)
class Logger {
    private logs: string[] = [];
    
    log(message: string): void {
        this.logs.push(message);
    }
    
    getLogs(): string[] {
        return this.logs;
    }
    
    clear(): void {
        this.logs = [];
    }
}

// Helper types
interface User {
    id: string;
    email: string;
    name: string;
}

interface Product {
    id: string;
    sku: string;
    price: number;
}