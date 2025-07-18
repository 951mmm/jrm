class Simple1Impl implements Runnable {
    @Override
    public void run() {
        System.out.println("run");
    }

    public static void main(String[] args) {
        System.out.println("hello!");
    }
}