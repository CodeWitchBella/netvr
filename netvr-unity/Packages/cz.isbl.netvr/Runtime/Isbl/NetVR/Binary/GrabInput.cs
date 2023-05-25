using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class GrabInput: IEquatable<GrabInput>, ICloneable {
        public ulong instance;
        public ulong session;
        public uint object_id;

        public GrabInput(ulong _instance, ulong _session, uint _object_id) {
            instance = _instance;
            session = _session;
            object_id = _object_id;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u64(instance);
            serializer.serialize_u64(session);
            serializer.serialize_u32(object_id);
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

        public static GrabInput Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            GrabInput obj = new GrabInput(
            	deserializer.deserialize_u64(),
            	deserializer.deserialize_u64(),
            	deserializer.deserialize_u32());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static GrabInput BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static GrabInput BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            GrabInput value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is GrabInput other && Equals(other);

        public static bool operator ==(GrabInput left, GrabInput right) => Equals(left, right);

        public static bool operator !=(GrabInput left, GrabInput right) => !Equals(left, right);

        public bool Equals(GrabInput other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!instance.Equals(other.instance)) return false;
            if (!session.Equals(other.session)) return false;
            if (!object_id.Equals(other.object_id)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + instance.GetHashCode();
                value = 31 * value + session.GetHashCode();
                value = 31 * value + object_id.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public GrabInput Clone() => (GrabInput)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
