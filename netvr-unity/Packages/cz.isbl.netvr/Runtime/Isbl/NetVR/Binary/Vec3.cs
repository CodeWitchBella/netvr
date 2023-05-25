using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class Vec3: IEquatable<Vec3>, ICloneable {
        public float x;
        public float y;
        public float z;

        public Vec3(float _x, float _y, float _z) {
            x = _x;
            y = _y;
            z = _z;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_f32(x);
            serializer.serialize_f32(y);
            serializer.serialize_f32(z);
            serializer.decrease_container_depth();
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

        public static Vec3 Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            Vec3 obj = new Vec3(
            	deserializer.deserialize_f32(),
            	deserializer.deserialize_f32(),
            	deserializer.deserialize_f32());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static Vec3 BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static Vec3 BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            Vec3 value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is Vec3 other && Equals(other);

        public static bool operator ==(Vec3 left, Vec3 right) => Equals(left, right);

        public static bool operator !=(Vec3 left, Vec3 right) => !Equals(left, right);

        public bool Equals(Vec3 other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!x.Equals(other.x)) return false;
            if (!y.Equals(other.y)) return false;
            if (!z.Equals(other.z)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + x.GetHashCode();
                value = 31 * value + y.GetHashCode();
                value = 31 * value + z.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public Vec3 Clone() => (Vec3)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
