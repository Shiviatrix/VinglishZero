def linear_search(numbers: list[int], target: int) -> int:
    index = 0
    for number in numbers:
        if number == target:
            return index
        index += 1
    return -1
