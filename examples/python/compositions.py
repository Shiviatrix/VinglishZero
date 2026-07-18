def filter_map_reduce(values: list[int]) -> int:
    result = 0
    for value in values:
        if value > 0:
            result = combine(result, transform(value))
    return result
