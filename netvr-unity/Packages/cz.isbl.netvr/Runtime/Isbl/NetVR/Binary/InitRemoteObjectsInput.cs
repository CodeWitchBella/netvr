using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class InitRemoteObjectsInput: IEquatable<InitRemoteObjectsInput>, ICloneable {
        public ulong instance;
        public ulong session;
        public Snapshot snapshot;

        public InitRemoteObjectsInput(ulong _instance, ulong _session, Snapshot _snapshot) {
            instance = _instance;
            session = _session;
            if (_snapshot == null) throw new ArgumentNullException(nameof(_snapshot));
            snapshot = _snapshot;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u64(instance);
            serializer.serialize_u64(session);
            snapshot.Serialize(serializer);
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

        public static InitRemoteObjectsInput Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            InitRemoteObjectsInput obj = new InitRemoteObjectsInput(
            	deserializer.deserialize_u64(),
            	deserializer.deserialize_u64(),
            	Snapshot.Deserialize(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static InitRemoteObjectsInput BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static InitRemoteObjectsInput BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            InitRemoteObjectsInput value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is InitRemoteObjectsInput other && Equals(other);

        public static bool operator ==(InitRemoteObjectsInput left, InitRemoteObjectsInput right) => Equals(left, right);

        public static bool operator !=(InitRemoteObjectsInput left, InitRemoteObjectsInput right) => !Equals(left, right);

        public bool Equals(InitRemoteObjectsInput other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!instance.Equals(other.instance)) return false;
            if (!session.Equals(other.session)) return false;
            if (!snapshot.Equals(other.snapshot)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + instance.GetHashCode();
                value = 31 * value + session.GetHashCode();
                value = 31 * value + snapshot.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public InitRemoteObjectsInput Clone() => (InitRemoteObjectsInput)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
