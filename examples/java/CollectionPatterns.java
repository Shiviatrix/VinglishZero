class CollectionPatterns {
    static int sum_values(int[] values) { int total = 0; for (int value : values) { total += value; } return total; }
    static int find_minimum(int[] values) { int minimum = values[0]; for (int value : values) { if (value < minimum) { minimum = value; } } return minimum; }
    static int count_if(int[] values) { int count = 0; for (int value : values) { if (value > 0) { count += 1; } } return count; }
    static boolean any_match(int[] values) { for (int value : values) { if (value > 0) { return true; } } return false; }
    static boolean all_positive(int[] values) { for (int value : values) { if (value <= 0) { return false; } } return true; }
    static int find_first(int[] values, int target) { for (int value : values) { if (value == target) { return value; } } return -1; }
    static int find_last(int[] values, int target) { int last = -1; for (int value : values) { if (value == target) { last = value; } } return last; }
    static int reverse_traversal(int[] values) { int total = 0; for (int value : values) { total += value; } return total; }
    static int prefix_sum(int[] values) { int total = 0; for (int value : values) { total += value; } return total; }
    static int frequency_counter(int[] values) { int count = 0; for (int value : values) { count += 1; } return count; }
    static int histogram(int[] values) { int count = 0; for (int value : values) { count += 1; } return count; }
    static int max_by(int[] values) { int best = values[0]; for (int value : values) { if (value > best) { best = value; } } return best; }
    static int min_by(int[] values) { int best = values[0]; for (int value : values) { if (value < best) { best = value; } } return best; }
    static int group_by(int[] values) { int count = 0; for (int value : values) { count += 1; } return count; }
    static int partition(int[] values) { int count = 0; for (int value : values) { if (value > 0) { count += 1; } } return count; }
    static int unique(int[] values) { int count = 0; for (int value : values) { if (value > 0) { count += 1; } } return count; }
    static int distinct(int[] values) { int count = 0; for (int value : values) { if (value > 0) { count += 1; } } return count; }
    static int zip_items(int[] values) { int total = 0; for (int value : values) { total += value; } return total; }
    static int flatten(int[] values) { int total = 0; for (int outer : values) { for (int inner : values) { total += inner; } } return total; }
    static boolean contains(int[] values, int target) { for (int value : values) { if (value == target) { return true; } } return false; }
}
