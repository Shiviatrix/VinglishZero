int transform(int value);

int mapper(int values[], int length) {
    int result = 0;
    for (int index = 0; index < length; index += 1) {
        result = transform(values[index]);
    }
    return result;
}
