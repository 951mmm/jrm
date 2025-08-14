import java.lang.annotation.*;

@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
@interface MyAnnotation {
  String value() default "test";

  Class<?> clazz() default Object.class;

  int num() default 42;

  MyEnum enu() default MyEnum.VALUE1;

  String[] array() default { "a", "b" };
}

enum MyEnum {
  VALUE1, VALUE2
}

@MyAnnotation(value = "runtime test", clazz = String.class, num = 99, enu = MyEnum.VALUE2, array = { "x", "y", "z" })
public class TestAnnotation {
  public static void main(String[] args) {
    System.out.println("Annotation test");
  }
}
