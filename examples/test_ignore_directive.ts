// Test file for similarity-ignore directive

// This function should be detected as similar to calculatePrice
function calculateTotal(items: any[]): number {
  return items.reduce((sum, item) => sum + item.price, 0);
}

// similarity-ignore
// This function is intentionally similar but should be ignored
function calculatePrice(products: any[]): number {
  return products.reduce((sum, p) => sum + p.price, 0);
}

/**
 * Calculate the sum of all values
 */
// similarity-ignore
function computeSum(values: any[]): number {
  return values.reduce((sum, v) => sum + v.price, 0);
}

// Regular function without ignore directive
function sumPrices(items: any[]): number {
  return items.reduce((total, item) => total + item.price, 0);
}

// Test with JSDoc and ignore directive
/**
 * This is a documented function
 * @param orders Array of orders
 * @returns Total price
 */
// similarity-ignore
function getTotalPrice(orders: any[]): number {
  return orders.reduce((sum, order) => sum + order.price, 0);
}

// Arrow function without ignore
const calculateCost = (items: any[]) => {
  return items.reduce((sum, item) => sum + item.price, 0);
};

// similarity-ignore
// Arrow function with ignore
const computeCost = (products: any[]) => {
  return products.reduce((sum, p) => sum + p.price, 0);
};

class PriceCalculator {
  // Method without ignore
  calculateTotal(items: any[]): number {
    return items.reduce((sum, item) => sum + item.price, 0);
  }

  // similarity-ignore
  // Method with ignore directive
  computeTotal(items: any[]): number {
    return items.reduce((sum, item) => sum + item.price, 0);
  }
}

// Export function without ignore
export function calculateOrderTotal(orders: any[]): number {
  return orders.reduce((sum, order) => sum + order.price, 0);
}

// similarity-ignore
export function computeOrderTotal(orders: any[]): number {
  return orders.reduce((sum, order) => sum + order.price, 0);
}
