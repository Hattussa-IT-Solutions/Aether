def format_name(name: str) -> str:
    """Format a name with title case."""
    return name.strip().title()

def clamp(value, min_val, max_val):
    return max(min_val, min(max_val, value))

def chunk_list(lst, size):
    return [lst[i:i + size] for i in range(0, len(lst), size)]
