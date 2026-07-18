def average(numbers: list[int]) -> float:
    total = 0
    count = 0
    for number in numbers:
        total += number
        count += 1
    return total / count
