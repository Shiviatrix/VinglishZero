int combine(int left, int right);

int reducer(int values[], int length) {
    int result = 0;
    for (int index = 0; index < length; index += 1) {
        result = combine(result, values[index]);
    }
    return result;
}
