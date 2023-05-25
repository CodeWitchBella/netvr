using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class Nothing: IEquatable<Nothing>, ICloneable {
        public byte value;

        public Nothing(byte _value) {
            value = _value;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u8(value);
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

        public static Nothing Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            Nothing obj = new Nothing(
            	deserializer.deserialize_u8());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static Nothing BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static Nothing BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            Nothing value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is Nothing other && Equals(other);

        public static bool operator ==(Nothing left, Nothing right) => Equals(left, right);

        public static bool operator !=(Nothing left, Nothing right) => !Equals(left, right);

        public bool Equals(Nothing other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!value.Equals(other.value)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + value.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public Nothing Clone() => (Nothing)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
