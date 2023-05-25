using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class InstanceAndSession: IEquatable<InstanceAndSession>, ICloneable {
        public ulong instance;
        public ulong session;

        public InstanceAndSession(ulong _instance, ulong _session) {
            instance = _instance;
            session = _session;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u64(instance);
            serializer.serialize_u64(session);
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

        public static InstanceAndSession Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            InstanceAndSession obj = new InstanceAndSession(
            	deserializer.deserialize_u64(),
            	deserializer.deserialize_u64());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static InstanceAndSession BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static InstanceAndSession BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            InstanceAndSession value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is InstanceAndSession other && Equals(other);

        public static bool operator ==(InstanceAndSession left, InstanceAndSession right) => Equals(left, right);

        public static bool operator !=(InstanceAndSession left, InstanceAndSession right) => !Equals(left, right);

        public bool Equals(InstanceAndSession other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!instance.Equals(other.instance)) return false;
            if (!session.Equals(other.session)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + instance.GetHashCode();
                value = 31 * value + session.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public InstanceAndSession Clone() => (InstanceAndSession)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
