int find_minimum(int numbers[], int length) {
    int minimum = numbers[0];
    for (int index = 0; index < length; index += 1) {
        if (numbers[index] < minimum) {
            minimum = numbers[index];
        }
    }
    return minimum;
}
