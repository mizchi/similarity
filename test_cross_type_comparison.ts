// Test for cross-type comparison (type alias vs interface vs type literal)

// Type alias
type UserType = {
    name: string;
    email: string;
    age: number;
};

// Interface with same structure
interface UserInterface {
    name: string;
    email: string;
    age: number;
}

// Type literal in variable (same structure)
const userLiteral: {
    name: string;
    email: string;
    age: number;
} = {
    name: "Test",
    email: "test@example.com",
    age: 25
};

// Function with type literal parameter (same structure)
function processUserLiteral(user: {
    name: string;
    email: string;
    age: number;
}): void {
    console.log(user);
}

// Arrow function with type literal return (same structure)
const getUserLiteral = (): {
    name: string;
    email: string;
    age: number;
} => {
    return { name: "User", email: "user@example.com", age: 30 };
};

// Slightly different structures for partial matching
type ProductType = {
    id: string;
    name: string;
    price: number;
    inStock?: boolean;
};

interface ProductInterface {
    id: string;
    name: string;
    price: number;
    stock?: number;  // Different optional property
}

// Another similar type with different name
type ItemType = {
    id: string;
    name: string;
    price: number;
    available?: boolean;
};

// Test self-comparison (should not detect as duplicate)
type SelfTestType = {
    uniqueField: string;
    anotherField: number;
};

// Using the same type in multiple places (should not be self-compared)
function useSelfTestType(data: SelfTestType): SelfTestType {
    return data;
}

const selfTestVar: SelfTestType = {
    uniqueField: "test",
    anotherField: 123
};

// Complex nested structures
interface NestedUserInterface {
    profile: {
        name: string;
        email: string;
        age: number;
    };
    settings: {
        theme: string;
        notifications: boolean;
    };
}

type NestedUserType = {
    profile: {
        name: string;
        email: string;
        age: number;
    };
    settings: {
        theme: string;
        notifications: boolean;
    };
};

// Type literal with nested structure
function processNestedUser(data: {
    profile: {
        name: string;
        email: string;
        age: number;
    };
    settings: {
        theme: string;
        notifications: boolean;
    };
}): void {
    console.log(data.profile);
}

// Testing method signatures
interface ServiceInterface {
    getData(): { status: string; data: any };
    setData(value: any): void;
}

type ServiceType = {
    getData(): { status: string; data: any };
    setData(value: any): void;
};

// Class with similar method signatures (for comparison)
class ServiceClass {
    getData(): { status: string; data: any } {
        return { status: "ok", data: null };
    }
    
    setData(value: any): void {
        console.log(value);
    }
}