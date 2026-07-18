def binary_search(numbers: list[int], target: int) -> int:
    low = 0
    high = 10
    while low <= high:
        middle = (low + high) // 2
        if numbers[middle] == target:
            return middle
        if numbers[middle] < target:
            low = middle + 1
        else:
            high = middle - 1
    return -1
