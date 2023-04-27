import random

N = 10
Q = 100_000
M = 1_000_000

L = [i + 1 for i in range(10)]
D = [100_000 for _ in range(10)]
QT = [(Q // 100, Q // 5, Q) for _ in range(10)]

print(f"{N} {Q} {M}")
print(" ".join(str(Li) for Li in L))
print(" ".join(str(Di) for Di in D))
print(" ".join(" ".join(str(Qtt) for Qtt in Qt) for Qt in QT))

weights = [
    1,
    1,
    1,
    2,
    3,
    2,
    2,
    1,
    1,
    1,
]

for i in range(M):
    b = random.choices(range(10), weights=weights)[0]
    if b < 5:
        page = random.randint(1, D[b] // 10)
    elif b < 7:
        page = random.randint(1, D[b] // 50)
    elif b < 9:
        page = (i % D[b]) + 1
    else:
        page = ((i * 2) % D[b]) + 1
    tenant = b + 1
    print(f"{tenant} {page}")

