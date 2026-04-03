data = list(range(50000))
total = 0
for item in data:
    if item % 3 == 0:
        total += item * 2
print(total)
