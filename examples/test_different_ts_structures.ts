// Test file to verify that clearly different TypeScript structures are not detected as similar

// Simple interface with two fields
interface Point2D {
  x: number;
  y: number;
}

// Complex interface with many fields
interface DatabaseConfig {
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  poolSize: number;
  timeout: number;
  ssl: boolean;
  certificate?: string;
  retryAttempts: number;
  retryDelay: number;
}

// Empty interface
interface Marker {}

// Single field type
type Id = string;

// Union type
type Status = 'pending' | 'active' | 'inactive' | 'deleted';

// Another simple interface that might look similar to Point2D
interface Coordinate {
  lat: number;
  lng: number;
}

// Complex type with nested structure
type ApiResponse<T> = {
  success: boolean;
  data: T;
  error?: {
    code: number;
    message: string;
    details?: string[];
  };
  metadata: {
    timestamp: number;
    version: string;
    requestId: string;
  };
};

// Class with methods (different from interfaces)
class UserService {
  private users: Map<string, any>;
  
  constructor() {
    this.users = new Map();
  }
  
  getUser(id: string) {
    return this.users.get(id);
  }
  
  addUser(id: string, data: any) {
    this.users.set(id, data);
  }
}

// Enum (different structure from interfaces)
enum Color {
  Red = '#FF0000',
  Green = '#00FF00',
  Blue = '#0000FF',
}

// Large configuration object
interface ApplicationConfig {
  appName: string;
  version: string;
  environment: 'dev' | 'staging' | 'production';
  features: {
    auth: boolean;
    analytics: boolean;
    notifications: boolean;
    darkMode: boolean;
  };
  api: {
    baseUrl: string;
    timeout: number;
    retries: number;
  };
  logging: {
    level: 'debug' | 'info' | 'warn' | 'error';
    file: string;
    console: boolean;
  };
}