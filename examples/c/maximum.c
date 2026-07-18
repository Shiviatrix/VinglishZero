int find_maximum(int numbers[], int length) {
    int maximum = numbers[0];
    int index = 0;
    while (index < length) {
        if (numbers[index] > maximum) {
            maximum = numbers[index];
        }
        index += 1;
    }
    return maximum;
}
