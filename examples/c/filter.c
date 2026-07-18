int filter_positive(int values[], int length) {
    int count = 0;
    for (int index = 0; index < length; index += 1) {
        if (values[index] > 0) {
            count += 1;
        }
    }
    return count;
}
