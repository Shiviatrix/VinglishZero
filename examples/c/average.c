double average(int numbers[], int length) {
    int total = 0;
    int count = 0;
    for (int index = 0; index < length; index += 1) {
        total += numbers[index];
        count += 1;
    }
    return total / count;
}
