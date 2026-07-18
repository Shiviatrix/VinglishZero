def find_minimum(numbers: list[int]) -> int:
    minimum = numbers[0]
    for number in numbers:
        if number < minimum:
            minimum = number
    return minimum
