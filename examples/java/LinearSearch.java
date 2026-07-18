class LinearSearch {
    static int linear_search(int[] numbers, int target) {
        for (int index = 0; index < numbers.length; index += 1) {
            if (numbers[index] == target) {
                return index;
            }
        }
        return -1;
    }
}
