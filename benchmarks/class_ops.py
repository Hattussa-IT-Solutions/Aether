import math
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def distance(self, other):
        dx = self.x - other.x
        dy = self.y - other.y
        return math.sqrt(dx * dx + dy * dy)
total = 0.0
for i in range(10000):
    p1 = Point(i * 1.0, i * 2.0)
    p2 = Point(i * 1.0 + 1.0, i * 3.0)
    total += p1.distance(p2)
print(total)
