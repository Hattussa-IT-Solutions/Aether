count = 0
s = "Hello World " * 100
for i in range(10000):
    if "World" in s: count += 1
print(count)
