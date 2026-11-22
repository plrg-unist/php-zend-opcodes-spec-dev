--TEST--
Readonly properties preserve public, protected and private setter visibility
--FILE--
<?php
class ParentValues {
    public public(set) readonly int $open;
    public readonly int $widened;
    public readonly int $implicit;
    public protected(set) readonly int $guarded;
    public private(set) readonly int $secret;

    public function initializePrivate(): void {
        $this->secret = 4;
    }
}

class ChildValues extends ParentValues {
    public public(set) readonly int $widened;

    public function initializeProtected(): void {
        $this->implicit = 2;
        $this->guarded = 3;
    }
}

$value = new ChildValues;
foreach (['open', 'widened', 'implicit', 'guarded', 'secret'] as $name) {
    $property = new ReflectionProperty($value, $name);
    $setter = $property->isPrivateSet() ? 'private' : ($property->isProtectedSet() ? 'protected' : 'public');
    try {
        $value->$name = 1;
        echo "$name $setter writable\n";
    } catch (Error) {
        echo "$name $setter restricted\n";
    }
}
$value->initializeProtected();
$value->initializePrivate();
echo $value->open, $value->widened, $value->implicit, $value->guarded, $value->secret, "\n";
--EXPECT--
open public writable
widened public writable
implicit protected restricted
guarded protected restricted
secret private restricted
11234
