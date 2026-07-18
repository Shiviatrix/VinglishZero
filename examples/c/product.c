int product(int numbers[], int length) {
    int result = 1;
    for (int index = 0; index < length; index += 1) {
        result *= numbers[index];
    }
    return result;
}
