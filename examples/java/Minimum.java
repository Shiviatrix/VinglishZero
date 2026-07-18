class Minimum {
    static int find_minimum(int[] numbers) {
        int minimum = numbers[0];
        for (int number : numbers) {
            if (number < minimum) {
                minimum = number;
            }
        }
        return minimum;
    }
}
