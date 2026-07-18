def sum_values(values: list[int]) -> int:
    total = 0
    for value in values: total += value
    return total

def find_minimum(values: list[int]) -> int:
    minimum = values[0]
    for value in values:
        if value < minimum: minimum = value
    return minimum

def count_if(values: list[int]) -> int:
    count = 0
    for value in values:
        if value > 0: count += 1
    return count

def any_match(values: list[int]) -> bool:
    for value in values:
        if value > 0: return True
    return False

def all_positive(values: list[int]) -> bool:
    for value in values:
        if value <= 0: return False
    return True

def find_first(values: list[int], target: int) -> int:
    for value in values:
        if value == target: return value
    return -1

def find_last(values: list[int], target: int) -> int:
    last = -1
    for value in values:
        if value == target: last = value
    return last

def reverse_traversal(values: list[int]) -> int:
    total = 0
    for value in values: total += value
    return total

def prefix_sum(values: list[int]) -> int:
    total = 0
    for value in values: total += value
    return total

def frequency_counter(values: list[int]) -> int:
    count = 0
    for value in values: count += 1
    return count

def histogram(values: list[int]) -> int:
    count = 0
    for value in values: count += 1
    return count

def max_by(values: list[int]) -> int:
    best = values[0]
    for value in values:
        if value > best: best = value
    return best

def min_by(values: list[int]) -> int:
    best = values[0]
    for value in values:
        if value < best: best = value
    return best

def group_by(values: list[int]) -> int:
    count = 0
    for value in values: count += 1
    return count

def partition(values: list[int]) -> int:
    count = 0
    for value in values:
        if value > 0: count += 1
    return count

def unique(values: list[int]) -> int:
    count = 0
    for value in values:
        if value > 0: count += 1
    return count

def distinct(values: list[int]) -> int:
    count = 0
    for value in values:
        if value > 0: count += 1
    return count

def zip_items(left: list[int], right: list[int]) -> int:
    total = 0
    for value in left: total += value
    return total

def flatten(values: list[list[int]]) -> int:
    total = 0
    for row in values:
        for value in row: total += value
    return total

def contains(values: list[int], target: int) -> bool:
    for value in values:
        if value == target: return True
    return False
