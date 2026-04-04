from src.models.user import User

def test_user_creation():
    u = User("Alice", 30)
    assert u.name == "Alice"
    assert u.age == 30

def test_user_greet():
    u = User("Bob", 25)
    assert u.greet() == "Hello, I'm Bob!"

def test_user_adult():
    assert User("Adult", 18).is_adult() == True
    assert User("Child", 10).is_adult() == False

def test_user_equality():
    u1 = User("Alice", 30)
    u2 = User("Alice", 30)
    u3 = User("Bob", 25)
    assert u1 == u2
    assert u1 != u3
