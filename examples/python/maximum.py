def find_maximum(numbers: list[int]) -> int:
    maximum = numbers[0]
    for number in numbers:
        if number > maximum:
            maximum = number
    return maximum
