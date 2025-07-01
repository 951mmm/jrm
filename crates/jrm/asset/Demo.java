class Demo implements Runnable {
    @Override
    public void run() {
        System.out.println("hello runnable!");
    }

    public static void main(String[] args) {
        System.out.println("hello jrm!");
    }
}