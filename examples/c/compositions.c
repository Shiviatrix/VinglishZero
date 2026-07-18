int transform(int value) { return value; }
int combine(int left, int right) { return left + right; }

int filter_map_reduce(int values[], int length) {
    int result = 0;
    for (int index = 0; index < length; index += 1) {
        if (values[index] > 0) {
            result = combine(result, transform(values[index]));
        }
    }
    return result;
}
