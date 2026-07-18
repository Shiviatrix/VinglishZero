def reducer(numbers: list[int]) -> int:
    result = 0
    for number in numbers:
        result = combine(result, number)
    return result
