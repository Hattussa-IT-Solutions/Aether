class User:
    def __init__(self, name: str, age: int):
        self.name = name
        self.age = age
    
    def greet(self) -> str:
        return f"Hello, I'm {self.name}!"
    
    def is_adult(self) -> bool:
        return self.age >= 18
    
    def to_dict(self) -> dict:
        return {"name": self.name, "age": self.age}
    
    def __str__(self):
        return f"User({self.name}, {self.age})"
    
    def __eq__(self, other):
        return self.name == other.name and self.age == other.age
