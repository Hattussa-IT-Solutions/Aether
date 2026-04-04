from src.models.user import User
from src.utils.helpers import format_name
import json

def main():
    users = [
        User("Alice", 30),
        User("Bob", 25),
        User("Charlie", 35),
    ]
    
    for user in users:
        print(f"{format_name(user.name)}: age {user.age}")
    
    # Serialize
    data = [u.to_dict() for u in users]
    print(json.dumps(data, indent=2))
    
    # Filter
    seniors = [u for u in users if u.age > 28]
    print(f"Seniors: {len(seniors)}")

if __name__ == "__main__":
    main()
