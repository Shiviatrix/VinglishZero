class Average {
    static double average(int[] numbers) {
        int total = 0;
        int count = 0;
        for (int number : numbers) {
            total += number;
            count += 1;
        }
        return total / count;
    }
}
