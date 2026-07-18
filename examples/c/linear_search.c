int linear_search(int numbers[], int length, int target) {
    for (int index = 0; index < length; index += 1) {
        if (numbers[index] == target) {
            return index;
        }
    }
    return -1;
}
