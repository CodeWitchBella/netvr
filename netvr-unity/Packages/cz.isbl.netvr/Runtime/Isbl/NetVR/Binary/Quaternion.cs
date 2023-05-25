using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class Quaternion: IEquatable<Quaternion>, ICloneable {
        public float x;
        public float y;
        public float z;
        public float w;

        public Quaternion(float _x, float _y, float _z, float _w) {
            x = _x;
            y = _y;
            z = _z;
            w = _w;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_f32(x);
            serializer.serialize_f32(y);
            serializer.serialize_f32(z);
            serializer.serialize_f32(w);
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

        public static Quaternion Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            Quaternion obj = new Quaternion(
            	deserializer.deserialize_f32(),
            	deserializer.deserialize_f32(),
            	deserializer.deserialize_f32(),
            	deserializer.deserialize_f32());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static Quaternion BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static Quaternion BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            Quaternion value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is Quaternion other && Equals(other);

        public static bool operator ==(Quaternion left, Quaternion right) => Equals(left, right);

        public static bool operator !=(Quaternion left, Quaternion right) => !Equals(left, right);

        public bool Equals(Quaternion other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!x.Equals(other.x)) return false;
            if (!y.Equals(other.y)) return false;
            if (!z.Equals(other.z)) return false;
            if (!w.Equals(other.w)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + x.GetHashCode();
                value = 31 * value + y.GetHashCode();
                value = 31 * value + z.GetHashCode();
                value = 31 * value + w.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public Quaternion Clone() => (Quaternion)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
