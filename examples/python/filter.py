def filter_positive(values: list[int]) -> list[int]:
    for value in values:
        if value > 0:
            emit(value)
