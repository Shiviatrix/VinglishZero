int calculate(int numbers[], int length) {
    int total = 0;
    int index = 0;
    while (index < length) {
        total += numbers[index];
        index += 1;
    }
    return total;
}
