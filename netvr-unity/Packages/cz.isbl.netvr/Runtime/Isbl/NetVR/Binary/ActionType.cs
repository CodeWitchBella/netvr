using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public abstract class ActionType: IEquatable<ActionType>, ICloneable {

        public abstract void Serialize(Serde.ISerializer serializer);

        public static ActionType Deserialize(Serde.IDeserializer deserializer) {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return Boolean.Load(deserializer);
                case 1: return Float.Load(deserializer);
                case 2: return Vector2f.Load(deserializer);
                case 3: return Pose.Load(deserializer);
                case 4: return VibrationOutput.Load(deserializer);
                case 5: return Unknown.Load(deserializer);
                default: throw new Serde.DeserializationException("Unknown variant index for ActionType: " + index);
            }
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static ActionType BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static ActionType BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            ActionType value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override int GetHashCode() {
            switch (this) {
            case Boolean x: return x.GetHashCode();
            case Float x: return x.GetHashCode();
            case Vector2f x: return x.GetHashCode();
            case Pose x: return x.GetHashCode();
            case VibrationOutput x: return x.GetHashCode();
            case Unknown x: return x.GetHashCode();
            default: throw new InvalidOperationException("Unknown variant type");
            }
        }
        public override bool Equals(object obj) => obj is ActionType other && Equals(other);

        public bool Equals(ActionType other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (GetType() != other.GetType()) return false;
            switch (this) {
            case Boolean x: return x.Equals((Boolean)other);
            case Float x: return x.Equals((Float)other);
            case Vector2f x: return x.Equals((Vector2f)other);
            case Pose x: return x.Equals((Pose)other);
            case VibrationOutput x: return x.Equals((VibrationOutput)other);
            case Unknown x: return x.Equals((Unknown)other);
            default: throw new InvalidOperationException("Unknown variant type");
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public ActionType Clone() => (ActionType)MemberwiseClone();

        object ICloneable.Clone() => Clone();


        public sealed class Boolean: ActionType, IEquatable<Boolean>, ICloneable {
            public Boolean() {
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                serializer.decrease_container_depth();
            }

            internal static Boolean Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                Boolean obj = new Boolean(
                	);
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is Boolean other && Equals(other);

            public static bool operator ==(Boolean left, Boolean right) => Equals(left, right);

            public static bool operator !=(Boolean left, Boolean right) => !Equals(left, right);

            public bool Equals(Boolean other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    return value;
                }
            }

        }

        public sealed class Float: ActionType, IEquatable<Float>, ICloneable {
            public Float() {
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(1);
                serializer.decrease_container_depth();
            }

            internal static Float Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                Float obj = new Float(
                	);
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is Float other && Equals(other);

            public static bool operator ==(Float left, Float right) => Equals(left, right);

            public static bool operator !=(Float left, Float right) => !Equals(left, right);

            public bool Equals(Float other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    return value;
                }
            }

        }

        public sealed class Vector2f: ActionType, IEquatable<Vector2f>, ICloneable {
            public Vector2f() {
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(2);
                serializer.decrease_container_depth();
            }

            internal static Vector2f Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                Vector2f obj = new Vector2f(
                	);
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is Vector2f other && Equals(other);

            public static bool operator ==(Vector2f left, Vector2f right) => Equals(left, right);

            public static bool operator !=(Vector2f left, Vector2f right) => !Equals(left, right);

            public bool Equals(Vector2f other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    return value;
                }
            }

        }

        public sealed class Pose: ActionType, IEquatable<Pose>, ICloneable {
            public Pose() {
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(3);
                serializer.decrease_container_depth();
            }

            internal static Pose Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                Pose obj = new Pose(
                	);
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is Pose other && Equals(other);

            public static bool operator ==(Pose left, Pose right) => Equals(left, right);

            public static bool operator !=(Pose left, Pose right) => !Equals(left, right);

            public bool Equals(Pose other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    return value;
                }
            }

        }

        public sealed class VibrationOutput: ActionType, IEquatable<VibrationOutput>, ICloneable {
            public VibrationOutput() {
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(4);
                serializer.decrease_container_depth();
            }

            internal static VibrationOutput Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                VibrationOutput obj = new VibrationOutput(
                	);
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is VibrationOutput other && Equals(other);

            public static bool operator ==(VibrationOutput left, VibrationOutput right) => Equals(left, right);

            public static bool operator !=(VibrationOutput left, VibrationOutput right) => !Equals(left, right);

            public bool Equals(VibrationOutput other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    return value;
                }
            }

        }

        public sealed class Unknown: ActionType, IEquatable<Unknown>, ICloneable {
            public Unknown() {
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(5);
                serializer.decrease_container_depth();
            }

            internal static Unknown Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                Unknown obj = new Unknown(
                	);
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is Unknown other && Equals(other);

            public static bool operator ==(Unknown left, Unknown right) => Equals(left, right);

            public static bool operator !=(Unknown left, Unknown right) => !Equals(left, right);

            public bool Equals(Unknown other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    return value;
                }
            }

        }
    }


} // end of namespace Isbl.NetVR.Binary
