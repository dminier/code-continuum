package backend.java;

public class DerivedClass extends BaseClass implements BaseInterface {
    private String derivedName;

    public DerivedClass() {
        this.derivedName = "DerivedClass";
    }

    @Override
    public void performAction() {
        System.out.println("Performing action in DerivedClass");
    }

    @Override
    public String getName() {
        return derivedName;
    }
}
