def map_values(values: list[int]) -> list[int]:
    for value in values:
        emit(transform(value))
