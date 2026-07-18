int binary_search(int numbers[], int length, int target) {
    int low = 0;
    int high = length - 1;
    while (low <= high) {
        int middle = (low + high) / 2;
        if (numbers[middle] == target) {
            return middle;
        }
        if (numbers[middle] < target) {
            low = middle + 1;
        } else {
            high = middle - 1;
        }
    }
    return -1;
}
