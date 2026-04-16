// Example MoonBit file with duplicate functions for testing similarity detection

pub fn process_items(items : Array[Int]) -> Array[Int] {
  let result : Array[Int] = []
  let mut i = 0
  while i < items.length() {
    if items[i] > 0 {
      result.push(items[i] * 2)
    }
    i = i + 1
  }
  result
}

pub fn handle_items(data : Array[Int]) -> Array[Int] {
  let output : Array[Int] = []
  let mut j = 0
  while j < data.length() {
    if data[j] > 0 {
      output.push(data[j] * 2)
    }
    j = j + 1
  }
  output
}

pub fn add(x : Int, y : Int) -> Int {
  x + y
}

pub fn multiply(x : Int, y : Int) -> Int {
  x * y
}

pub fn greet(name : String) -> String {
  "Hello, " + name + "!"
}
