// Test file with many similar type definitions to verify fingerprint optimization

interface User1 {
    id: number;
    name: string;
    email: string;
    age: number;
}

interface User2 {
    userId: number;
    userName: string;
    userEmail: string;
    userAge: number;
}

interface Person1 {
    id: number;
    fullName: string;
    emailAddress: string;
    yearsOld: number;
}

type Customer1 = {
    customerId: number;
    customerName: string;
    contactEmail: string;
    age: number;
};

type Account1 = {
    accountId: number;
    accountName: string;
    email: string;
    accountAge: number;
};

// Different structure
interface Product {
    productId: string;
    productName: string;
    price: number;
    inStock: boolean;
}

interface Order {
    orderId: string;
    items: string[];
    totalAmount: number;
    isPaid: boolean;
}

// More similar types
interface Employee1 {
    employeeId: number;
    name: string;
    email: string;
    department: string;
}

interface Staff1 {
    staffId: number;
    fullName: string;
    emailAddr: string;
    dept: string;
}

type Worker1 = {
    workerId: number;
    workerName: string;
    workerEmail: string;
    section: string;
};

// Generate many similar types for performance testing
interface DataModel1 {
    id: string;
    createdAt: Date;
    updatedAt: Date;
    version: number;
}

interface DataModel2 {
    modelId: string;
    created: Date;
    updated: Date;
    versionNumber: number;
}

interface DataModel3 {
    identifier: string;
    creationDate: Date;
    modificationDate: Date;
    revision: number;
}

type DataEntity1 = {
    entityId: string;
    createdDate: Date;
    modifiedDate: Date;
    ver: number;
};

type DataEntity2 = {
    id: string;
    createTime: Date;
    updateTime: Date;
    version: number;
};

// Nested structures
interface ComplexUser1 {
    id: number;
    profile: {
        name: string;
        email: string;
        phone: string;
    };
    settings: {
        theme: string;
        notifications: boolean;
    };
}

interface ComplexUser2 {
    userId: number;
    userProfile: {
        fullName: string;
        emailAddress: string;
        phoneNumber: string;
    };
    preferences: {
        uiTheme: string;
        enableNotifications: boolean;
    };
}

// Generic types
interface Response1<T> {
    data: T;
    status: number;
    message: string;
}

interface Response2<T> {
    result: T;
    statusCode: number;
    statusMessage: string;
}

interface ApiResponse<T> {
    payload: T;
    code: number;
    description: string;
}